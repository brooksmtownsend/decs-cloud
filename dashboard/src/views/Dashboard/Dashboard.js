import React, { Component } from 'react';
import {
  Col,
  Row,
} from 'reactstrap';
import ShardList from './ShardList';
import SystemList from './SystemList';

import ResClient from 'resclient';

const client = new ResClient('/resgate')

class Dashboard extends Component {
  constructor(props) {
    super(props);

    this.toggle = this.toggle.bind(this);
    this.onRadioBtnClick = this.onRadioBtnClick.bind(this);

    this.state = {
      dropdownOpen: false,
      radioSelected: 2,
      systems: null,
      shards: null
    };
  }

  onUpdate = () => {
    this.setState({})
  }

  componentDidMount() {
    this.getShards();
    this.getSystems();
  }

  getShards() {
    client.get('decs.shards').then(shards => {
      shards.on('add', this.onUpdate)
      shards.on('remove', this.onUpdate)
      this.setState({ shards })
    }).catch(err => {
      console.log(err)
    })
  }

  getSystems() {
    client.get('decs.systems').then(systems => {
      systems.on('add', this.onUpdate)
      systems.on('remove', this.onUpdate)
      this.setState({ systems })
    }).catch(err => {
      console.log(err)
    })
  }

  toggle() {
    this.setState({
      dropdownOpen: !this.state.dropdownOpen,
    });
  }

  onRadioBtnClick(radioSelected) {
    this.setState({
      radioSelected: radioSelected,
    });
  }

  loading = () => <div className="animated fadeIn pt-1 text-center">Loading...</div>

  render() {

    return (
      <div className="animated fadeIn">
        <Row>
          <Col>

            {
              this.state.systems ?
                <SystemList systems={this.state.systems}></SystemList>
                : null
            }

          </Col>

        </Row>

        {
          this.state.shards ?
            <ShardList shards={this.state.shards}></ShardList>
            : null
        }


                
      </div>
    );
  }
}

export default Dashboard;
