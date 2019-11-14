import React, { Component } from 'react';
import {
  Card,
  CardBody,
  CardHeader,
  Col,
  Row,
} from 'reactstrap';
import ShardList from './ShardList';
import SystemList from './SystemList';
import LeaderBoard from './Leaderboard';

import ResClient from 'resclient';

const client = new ResClient('/resgate')
//const client = new ResClient("ws://localhost:8080/resgate");

class Dashboard extends Component {
  constructor(props) {
    super(props);

    this.toggle = this.toggle.bind(this);
    this.onRadioBtnClick = this.onRadioBtnClick.bind(this);

    this.state = {
      dropdownOpen: false,
      radioSelected: 2,
      systems: null,
      shards: null,
      lbitems: null
    };
  }

  onUpdate = () => {
    this.setState({})
  }

  componentDidMount() {
    this.getShards();
    this.getSystems();
    this.getLeaderboard();
  }

  getLeaderboard() {
    client.get('decs.mainworld.leaderboard').then(lbitems => {
      lbitems.on('add', this.onUpdate);
      lbitems.on('remove', this.onUpdate);
      this.setState({ lbitems })
    }).catch(err => {
      console.log(err)
    })
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
            <Row>
              <Col>
                <Card>
                  <CardHeader>
                    Systems
              </CardHeader>
                  <CardBody>
                    {
                      this.state.systems ?
                        <SystemList systems={this.state.systems}></SystemList>
                        : null
                    }
                  </CardBody>
                </Card>
              </Col>
            </Row>
          </Col>
        </Row>

        <Row>
          <Col>
            <Card>
              <CardHeader>
                Shards
              </CardHeader>
              <CardBody>
                <Row>
                  <Col>

                    {
                      this.state.shards ?
                        <ShardList shards={this.state.shards}></ShardList>
                        : null
                    }

                  </Col>
                </Row>
              </CardBody>
            </Card>
          </Col>
        </Row>

        <Row>
        <Col>
          <Card>
            <CardHeader>
              Leaderboard - mainworld
              </CardHeader>
            <CardBody>
              <Row>
                <Col>
                    {
                      this.state.lbitems ?
                         <LeaderBoard items={this.state.lbitems}></LeaderBoard>
                         :null
                    }
                </Col>
              </Row>
            
            </CardBody>
          </Card>
        </Col>
      </Row>


      </div>
    );
  }
}

export default Dashboard;
